use juzo_core::{
    application::{JuzoAnswer, UserIds},
    common::emojis::{smail_pensil, smail_tick},
};
use sea_orm::{ConnectionTrait, raw_sql};

use super::super::*;

async fn add_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
    show: bool,
    is_parent: bool,
) -> HandlerResult<()> {
    // SAFETY: TBA will never return None in message.from().
    let my_ids: UserIds = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id
    .into();

    if !my_ids.is_creator_bot() {
        return Ok(());
    }

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<2>(text);
    let comment = &text[result.first_line];

    if comment.is_empty() {
        return Ok(());
    }

    let (module_ids, rank): (i16, u8) = match args {
        ArgsResult::Some([a1, a2], 2) => {
            let Ok(module_ids) = text[a1].parse::<i16>() else {
                return Ok(());
            };

            let Ok(rank) = text[a2].parse::<u8>() else {
                return Ok(());
            };

            (module_ids, rank)
        }
        ArgsResult::Some([a1, _], 1) => {
            let Ok(module_ids) = text[a1].parse::<i16>() else {
                return Ok(());
            };

            (module_ids, 0)
        }
        _ => return Ok(()),
    };

    if module_ids <= 0 {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Такого модуля не будет существовать.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    let row = if show {
        db.query_one_raw(raw_sql!(
            Postgres,
            r#"
            INSERT INTO b2 (
                module_ids,
                name,
                is_parent,
                show,
                rank
            )
            SELECT
                {module_ids},
                {comment},
                {is_parent},
                true,
                {rank}
            WHERE NOT EXISTS (
                SELECT 1
                FROM b2
                WHERE module_ids = {module_ids}
                   OR name = {comment}
            )
            RETURNING true AS result
            "#
        ))
        .await
        .unwrap_or(None)
    } else {
        db.query_one_raw(raw_sql!(
            Postgres,
            r#"
            INSERT INTO b2 (
                module_ids,
                name,
                is_parent,
                show,
                rank
            )
            VALUES (
                {module_ids},
                {comment},
                {is_parent},
                false,
                {rank}
            )
            RETURNING true AS result
            "#
        ))
        .await
        .unwrap_or(None)
    };

    let inserted = row
        .map(|row| {
            row.try_get::<bool>("", "result")
                .unwrap_or(false)
        })
        .unwrap_or(false);

    if !inserted {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Нашёлся конфликтный модуль.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    }

    let status = if is_parent {
        "Раздел"
    } else {
        "Модуль"
    };

    bot.send(JuzoAnswer::message(&message).text(format!(
        "{0} {status} «<code>{comment}</code>» (<code>{module_ids}</code>; {show}) теперь \
         доступен с {rank} ранга",
        smail_tick(true)
    )))
    .await?;

    Ok(())
}

pub async fn add(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    add_core(bot, message, db, result, true, false).await
}

pub async fn add_parent(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    add_core(bot, message, db, result, true, true).await
}

pub async fn show_add(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    add_core(bot, message, db, result, false, false).await
}

pub async fn show_add_parent(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    add_core(bot, message, db, result, false, true).await
}
