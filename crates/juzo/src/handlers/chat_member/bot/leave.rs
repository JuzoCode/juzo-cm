use juzo_core::db::chat::prelude::Chat;
use sea_orm::{ConnectionTrait, DatabaseBackend, DbConn, EntityTrait, Statement};
use telers::methods::LeaveChat;

use super::super::*;

pub async fn yes(
    bot: Bot,
    member: ChatMemberUpdated,
    Extension(db): Extension<DbConn>,
) -> HandlerResult {
    let chat_ids = member.chat.id();

    let limit = match db
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM c
                WHERE skip = false
                    and bot_admin = true
                OFFSET 19
                LIMIT 1
            ) AS allowed
            "#,
        ))
        .await
        .ok()
        .flatten()
    {
        Some(row) => row
            .try_get::<bool>("", "allowed")
            .unwrap_or_default(),
        None => false,
    };

    if !limit {
        return Ok(telers::event::EventReturn::Skip);
    }

    let _ = Chat::delete_by_id(chat_ids)
        .exec(&db)
        .await;

    bot.send(LeaveChat::new(chat_ids))
        .await?;

    Ok(telers::event::EventReturn::Finish)
}
