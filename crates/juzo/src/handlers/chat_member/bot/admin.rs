use core::intrinsics::unreachable;

use sea_orm::{ConnectionTrait, raw_sql};
use telers::{
    enums::ParseMode,
    event::EventReturn,
    methods::{GetChatAdministrators, SendMessage},
    types::{Chat, ChatMember},
};

use super::super::*;

pub async fn added(
    bot: Bot,
    member: ChatMemberUpdated,
    Extension(db): Extension<DbConn>,
) -> HandlerResult<EventReturn> {
    let Chat::Supergroup(chat) = *member.chat else {
        // SAFETY: The ChatType filter allows processing only of "Supergroup".
        unsafe { unreachable() }
    };

    // SAFETY: The "None" value only occurs for users.
    let full_name = unsafe {
        chat.title
            .as_deref()
            .unwrap_unchecked()
    };

    let Ok(result) = db
        .execute_raw(raw_sql!(
            Postgres,
            r#"
            INSERT INTO c (
                chat_ids,
                bot_admin,
                full_name
            )
            VALUES (
                {chat.id},
                true,
                {full_name}
            )
            ON CONFLICT (chat_ids) DO UPDATE SET
                bot_admin = true
            WHERE c.bot_admin = false
            "#
        ))
        .await
    else {
        return Ok(EventReturn::Finish);
    };

    if result.rows_affected() == 0 {
        return Ok(EventReturn::Finish);
    }

    let members = bot
        .send(GetChatAdministrators::new(chat.id))
        .await?;

    if let Some(owner_ids) = members
        .iter()
        .find_map(|member| match member {
            ChatMember::Creator(owner) => Some(owner.user.id),
            _ => None,
        })
    {
        let _ = db
            .execute_raw(raw_sql!(
                Postgres,
                r#"
                UPDATE c SET owner_ids = {owner_ids}
                    WHERE chat_ids = {chat.id};
                "#
            ))
            .await;

        let _ = db
            .execute_raw(raw_sql!(
                Postgres,
                r#"
                INSERT INTO c4 (
                    user_ids,
                    chat_ids,
                    peer_ids,
                    rank,
                    sms_ids
                )
                VALUES (
                    {owner_ids},
                    {chat.id},
                    {owner_ids},
                    6,
                    0
                )
                "#
            ))
            .await;
    }

    bot.send(
        SendMessage::new(
            chat.id,
            "🗓 <b>Я так рад</b>, что меня добавили в статус администрации!\n\nТеперь у меня \
             <b>много возможностей</b>. Какие именно? Можно посмотреть в <a \
             href='https://teletype.in/@juzo_cm/commands'>нашей статье</a>.\nЕсли вам непонятны какие-то \
             моменты, можно обратиться в <a href='https://t.me/juzo_cm_chat'>чат поддержки</a>",
        )
        .parse_mode(ParseMode::HTML),
    )
    .await?;

    Ok(EventReturn::Finish)
}
