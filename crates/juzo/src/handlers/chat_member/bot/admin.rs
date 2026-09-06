use sea_orm::{ConnectionTrait, raw_sql};
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
    let full_name = unsafe {
        member
            .chat
            .title()
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
            VALUES ({chat_ids}, true, {full_name})
            ON CONFLICT (chat_ids) DO UPDATE SET
                bot_admin = true
            WHERE bot_admin = false
            "#
        ))
        .await
    else {
        return Ok(());
    };

    if result.rows_affected() == 0 {
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
        let _ = db
            .execute_raw(raw_sql!(
                Postgres,
                r#"
                UPDATE c SET owner_ids = {owner_ids}
                    WHERE chat_ids = {chat_ids};

                INSERT INTO c4 (
                    user_ids,
                    chat_ids,
                    peer_ids,
                    rank,
                    sms_ids
                )
                VALUES (
                    {owner_ids},
                    {chat_ids},
                    {owner_ids},
                    6,
                    0
                )
                "#
            ))
            .await;
    }

    bot.send(SendMessage::new(chat_ids, "Я так рад что я стал администратором"))
        .await?;

    Ok(())
}
