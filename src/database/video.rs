use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl as _;

use crate::{DbConnection, database::DbError, models::Video, schema::video::dsl::*};

pub async fn create_or_update_video(
    conn: &mut DbConnection,
    video_: &Video,
) -> Result<(), DbError> {
    diesel::insert_into(video)
        .values(video_)
        .on_conflict(id)
        .do_update()
        .set(video_)
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn get_video_by_id(
    conn: &mut DbConnection,
    video_id: &str,
) -> Result<Option<Video>, DbError> {
    video
        .filter(id.eq(video_id.to_string()))
        .select(Video::as_select())
        .first(conn)
        .await
        .optional()
}
