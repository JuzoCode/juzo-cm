use juzo_core::db::chat::{chat, prelude::Chat};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, prelude::Expr};
use telers::methods::SendMessage;

use super::super::*;

pub async fn set(
    bot: Bot,
    member: ChatMemberUpdated,
    Extension(db): Extension<DbConn>,
) -> HandlerResult<()> {
    let Ok(result) = Chat::update_many()
        .col_expr(chat::Column::BotAdmin, Expr::value(true))
        .filter(chat::Column::ChatIds.eq(member.chat.id()))
        .filter(chat::Column::BotAdmin.eq(false))
        .exec(&db)
        .await
    else {
        return Ok(());
    };

    if result.rows_affected == 0 {
        return Ok(());
    }

    bot.send(SendMessage::new(member.chat.id(), "Я так рад что я стал админисратором"))
        .await?;
    Ok(())
}
