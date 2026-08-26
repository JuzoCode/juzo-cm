use juzo_core::db::chat::{chat, moder, prelude::Chat};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, prelude::Expr};
use telers::{
    methods::{GetChatAdministrators, SendMessage},
    types::ChatMember,
};

use super::super::*;

pub async fn set(
    bot: Bot,
    member: ChatMemberUpdated,
    Extension(db): Extension<DbConn>,
) -> HandlerResult<()> {
    let chat_ids = member.chat.id();

    let Ok(result) = Chat::update_many()
        .col_expr(chat::Column::BotAdmin, Expr::value(true))
        .filter(chat::Column::ChatIds.eq(chat_ids))
        .filter(chat::Column::BotAdmin.eq(false))
        .exec(&db)
        .await
    else {
        return Ok(());
    };

    if result.rows_affected == 0 {
        return Ok(());
    }

    let members = bot
        .send(GetChatAdministrators::new(chat_ids))
        .await?;

    if let Some(owner_ids) = members
        .iter()
        .find_map(|member| match member {
            ChatMember::Creator(owner) => Some(owner.user.id),
            _ => None,
        })
    {
        let _ = Chat::update_many()
            .col_expr(chat::Column::OwnerIds, Expr::value(owner_ids))
            .filter(chat::Column::ChatIds.eq(chat_ids))
            .exec(&db)
            .await;

        let _ = moder::ActiveModel {
            user_ids: Set(owner_ids.into()),
            chat_ids: Set(chat_ids.into()),
            peer_ids: Set(owner_ids.into()),
            rank: Set(6),
            sms_ids: Set(0),
            ..Default::default()
        }
        .insert(&db)
        .await;
    }

    bot.send(SendMessage::new(chat_ids, "Я так рад что я стал администратором"))
        .await?;

    Ok(())
}
