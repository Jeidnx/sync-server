use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl as _;

use crate::{
    DbConnection,
    database::{DbError, channel::create_or_update_channel, video::create_or_update_video},
    models::{Channel, Video, WatchHistoryItem, WatchedState},
    schema::{channel, video, watch_history::dsl::*},
};

const PAGE_SIZE: i64 = 50;

pub async fn get_watch_history_by_account_id(
    conn: &mut DbConnection,
    account_id_: &str,
    page_num: u32,
    status: &Option<WatchedState>,
    sort_by_date_ascending: bool,
) -> Result<Vec<(WatchHistoryItem, Video, Channel)>, DbError> {
    // https://github.com/diesel-rs/diesel/issues/455
    let mut query = watch_history
        .filter(account_id.eq(account_id_)).into_boxed();

    if let Some(status) = &status {
        query = query.filter(watched_state.eq(status));
    }

    if sort_by_date_ascending {
        query = query.order(added_date.asc())
    } else {
        query = query.order(added_date.desc())
    }

    query
        .offset(PAGE_SIZE * (page_num - 1) as i64)
        .limit(PAGE_SIZE)
        .inner_join(video::table.inner_join(channel::table))
        .select((WatchHistoryItem::as_select(), Video::as_select(), Channel::as_select()))
        .load(conn)
        .await
}

pub async fn add_video_to_watch_history(
    conn: &mut DbConnection,
    watch_history_item_: &WatchHistoryItem,
    video_: &Video,
    channel_: &Channel,
) -> Result<(), DbError> {
    create_or_update_channel(conn, channel_).await?;
    create_or_update_video(conn, video_).await?;

    diesel::insert_into(watch_history)
        .values(watch_history_item_)
        .on_conflict((video_id, account_id))
        .do_update()
        .set(watch_history_item_)
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn remove_video_from_watch_history(
    conn: &mut DbConnection,
    account_id_: &str,
    video_id_: &str,
) -> Result<(), DbError> {
    diesel::delete(
        watch_history.filter(
            account_id
                .eq(account_id_.to_string())
                .and(video_id.eq(video_id_.to_string())),
        ),
    )
    .execute(conn)
    .await?;

    Ok(())
}
