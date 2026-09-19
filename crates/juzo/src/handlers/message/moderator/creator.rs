use juzo_core::{
    common::emojis::{smail_pensil, smail_tick},
    domain::{UserModel, UserModelExt},
    gender,
};
use sea_orm::{ConnectionTrait, raw_sql};
use telers::{methods::GetChatAdministrators, types::ChatMember};

use super::super::*;

pub async fn repair(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let chat_ids = message.chat().id();
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
            .id
    };

    let row = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            SELECT
                c.owner_ids,
                u.full_name,
                COALESCE(
                    'https://t.me/' || u.username,
                    'tg://openmessage/user_id=' || c.owner_ids::text
                ) AS link,
                u.gender
            FROM c
            JOIN u
                ON u.user_ids = c.owner_ids
            WHERE c.chat_ids = {chat_ids}
                AND c.owner_ids <> 0
            "#
        ))
        .await;

    let (user_ids, full_name, link, gender) = if let Ok(Some(row)) = row {
        unsafe {
            (
                row.try_get::<i64>("", "owner_ids")
                    .unwrap_unchecked(),
                row.try_get::<String>("", "full_name")
                    .unwrap_unchecked(),
                row.try_get::<String>("", "link")
                    .unwrap_unchecked(),
                row.try_get::<i16>("", "gender")
                    .unwrap_unchecked(),
            )
        }
    } else {
        let members = bot
            .send(GetChatAdministrators::new(chat_ids))
            .await?;

        let Some(owner) = members
            .iter()
            .find_map(|member| match member {
                ChatMember::Creator(owner) => Some(owner.user.as_ref()),
                _ => None,
            })
        else {
            bot.send(JuzoAnswer::message(&message).text(format!(
                "{0} Трон стоит, царя не видно.",
                smail_pensil(true)
            )))
            .await?;
            return Ok(());
        };

        let owner = UserModel::new(&db, owner).await;

        let _ = db
            .execute_raw(raw_sql!(
                Postgres,
                r#"
                UPDATE c
                SET owner_ids = {owner.ids}
                WHERE chat_ids = {chat_ids}
                "#
            ))
            .await;

        (
            owner.ids.0,
            owner.full_name(),
            owner.link().into(),
            owner.gender,
        )
    };

    if my_ids != user_ids {
        let module = ModuleChecker::new(&bot, &db);
        let true = module
            .check::<42>(ModuleAccess::M(&message))
            .await
        else {
            return Ok(());
        };

        let g1 = gender!(gender => ["ьницей", "ем"]);

        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Ого, вижу, амбиции у вас что надо. Но увы, <b>создател{g1} чата является <a \
             href='{1}'>{2}</a></b>.",
            smail_pensil(true),
            link,
            full_name
        )))
        .await?;
        return Ok(());
    }

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
                {user_ids},
                {chat_ids},
                {user_ids},
                6,
                0
            )
            ON CONFLICT DO NOTHING
            "#
        ))
        .await;

    let g1 = gender!(gender => ["ьнице", "ю"]);

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} Основател{g1} чата возвращены права владения",
        smail_tick(true)
    )))
    .await?;

    Ok(())
}
