use juzo_core::db::chat::{chat, prelude::Chat};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
use telers::types::{MessageCommunityChatAdded, MessageCommunityChatRemoved};

use super::super::{DbConn, Extension, HandlerResult};

pub async fn added(
    message: MessageCommunityChatAdded,
    Extension(db): Extension<DbConn>,
) -> HandlerResult<()> {
    let _ = Chat::update_many()
        .col_expr(
            chat::Column::Community,
            Expr::value(
                message
                    .community_chat_added
                    .community
                    .id,
            ),
        )
        .filter(chat::Column::ChatIds.eq(message.chat.id()))
        .exec(&db)
        .await;

    Ok(())
}

pub async fn removed(
    message: MessageCommunityChatRemoved,
    Extension(db): Extension<DbConn>,
) -> HandlerResult<()> {
    let _ = Chat::update_many()
        .col_expr(chat::Column::Community, Expr::value(0))
        .filter(chat::Column::ChatIds.eq(message.chat.id()))
        .exec(&db)
        .await;

    Ok(())
}
