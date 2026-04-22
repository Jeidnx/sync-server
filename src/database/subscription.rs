use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper, associations::HasTable,
};
use diesel_async::RunQueryDsl;

use crate::{
    DbConnection,
    database::{DbError, channel::create_or_update_channel},
    models::{self, Channel, Subscription},
    schema::{channel::dsl::channel, subscription::dsl::*},
};

pub async fn get_subscriptions_by_user_id(
    conn: &mut DbConnection,
    user_id_: &str,
) -> Result<Vec<models::Channel>, DbError> {
    let item = subscription
        .filter(user_id.eq(user_id_.to_string()))
        .inner_join(channel::table())
        .select(models::Channel::as_select())
        .load::<models::Channel>(conn)
        .await?;

    Ok(item)
}

pub async fn add_subscription_by_user_id(
    conn: &mut DbConnection,
    channel_: &Channel,
    user_id_: &str,
) -> Result<(), DbError> {
    create_or_update_channel(conn, channel_).await?;

    let new_subscription = Subscription {
        user_id: user_id_.to_string(),
        channel_id: channel_.id.clone(),
    };
    diesel::insert_into(subscription)
        .values(&new_subscription)
        .on_conflict_do_nothing()
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn remove_subscription_by_user_id(
    conn: &mut DbConnection,
    channel_id_: &str,
    user_id_: &str,
) -> Result<(), DbError> {
    diesel::delete(
        subscription.filter(
            user_id
                .eq(user_id_.to_string())
                .and(channel_id.eq(channel_id_.to_string())),
        ),
    )
    .execute(conn)
    .await?;

    Ok(())
}
