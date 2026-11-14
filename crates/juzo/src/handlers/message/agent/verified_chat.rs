use juzo_core::{
    common::emojis::{smail_cross, smail_pensil, smail_tick},
    db::agent::{agent, prelude::Agent},
};
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect, raw_sql};
use telers::methods::SendMessage;

use super::super::*;

async fn _verified(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
    verified: bool,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let Ok(Some(true)) = Agent::find_by_id(my_ids)
        .select_only()
        .column(agent::Column::Agent)
        .into_tuple::<bool>()
        .one(&db)
        .await
    else {
        return Ok(());
    };

    let chat_ids = message.chat().id();

    let Ok(_) = db
        .execute_raw(raw_sql!(
            Postgres,
            r#"
            INSERT INTO c3 (chat_ids, verified, trusted)
            VALUES (
                {chat_ids},
                {verified}::boolean,
                CASE
                    WHEN {verified}::boolean THEN EXTRACT(EPOCH FROM NOW())::bigint
                    ELSE 0
                END
            )
            ON CONFLICT (chat_ids) DO UPDATE SET
                verified = EXCLUDED.verified,
                trusted = CASE
                    WHEN EXCLUDED.verified THEN EXCLUDED.trusted
                    ELSE c3.trusted
                END;
            "#
        ))
        .await
    else {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} Статус доверенного чата не был изменён.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    if verified {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} Теперь этот чат является доверенным",
            smail_tick(true)
        )))
        .await?;
    } else {
        bot.send(JuzoAnswer::<SendMessage>::to(&message).text(format!(
            "{0} Доверенность чата на нуле... Ну, извините уж",
            smail_cross(true)
        )))
        .await?;
    }

    Ok(())
}

pub async fn yes(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    _verified(bot, message, db, result, true).await
}

pub async fn no(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    _verified(bot, message, db, result, false).await
}
