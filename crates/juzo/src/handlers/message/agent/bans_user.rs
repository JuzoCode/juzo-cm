use core::{fmt::Write, intrinsics::unreachable};

use chrono::{Local, TimeZone};
use juzo_core::{
    application::{UserIndex, UserModel},
    common::inflection::plur_mark,
    db::agent::{agent, prelude::Agent},
};
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect, raw_sql};

use super::super::*;

async fn show_core(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
    full: bool,
) -> HandlerResult<()> {
    // SAFETY: TBA will never return None in message.from().
    let my_ids = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .id;

    let Ok(Some((is_agent, is_spam))) = Agent::find_by_id(my_ids)
        .select_only()
        .columns([agent::Column::Agent, agent::Column::Spam])
        .into_tuple::<(bool, bool)>()
        .one(&db)
        .await
    else {
        return Ok(());
    };

    if !is_agent && (!is_spam || full) {
        return Ok(());
    }

    let user_ind = UserIndex::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);

    let user: UserModel = match args {
        ArgsResult::Some([a1], _) => {
            let Ok(found_user) = user_ind
                .search_user(&text[a1])
                .await
            else {
                return Ok(());
            };
            found_user
        }
        // SAFETY: TBA will never return None in message.from().
        ArgsResult::None => unsafe {
            if let Some(r) = message.reply_to_message() {
                r.from()
                    .unwrap_unchecked()
                    .into()
            } else {
                return Ok(());
            }
        },
        ArgsResult::Unk => return Ok(()),
    };

    let blocks = db
        .query_all_raw(raw_sql!(
            Postgres,
            r#"
            SELECT
                u.full_name,
                COALESCE(
                    'https://t.me/' || u.username,
                    'tg://openmessage/user_id=' || a3.agents_ids::text
                ) AS link,
                a3.added,
                a3.reason,
                a3.function
            FROM a3
            LEFT JOIN u
                ON u.user_ids = a3.agents_ids
            WHERE a3.user_ids = {user.ids}
                AND function <> 0 
            "#
        ))
        .await
        .unwrap_or_default();

    let rows = db
        .query_all_raw(raw_sql!(
            Postgres,
            r#"
            SELECT
                u.full_name,
                COALESCE(
                    'https://t.me/' || u.username,
                    'tg://openmessage/user_id=' || a.agents_ids::text
                ) AS link,
                a.removed,
                a.count,
                a.reason,
                a.function
            FROM (
                SELECT DISTINCT ON (function)
                    agents_ids,
                    removed,
                    reason,
                    function,
                    COUNT(*) OVER (PARTITION BY function) AS count
                FROM a2
                WHERE
                    user_ids = {user.ids}
                    AND function IN (1, 2)
                ORDER BY
                    function,
                    removed DESC,
                    ids DESC
            ) a
            LEFT JOIN u
                ON u.user_ids = a.agents_ids
            "#
        ))
        .await
        .unwrap_or_default();

    if blocks.is_empty() && rows.is_empty() {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "<b>Баны <a href='{0}'>{1}</a>.</b>\n\n🗓 Абсолютно чист и <b>не имеет выносов</b> в \
             базах Джузо",
            user.link(),
            user.full_name()
        )))
        .await?;
        return Ok(());
    }

    let mut text = format!(
        "<b>Баны <a href='{0}'>{1}</a>.</b>\n",
        user.link(),
        user.full_name()
    );

    for row in &rows {
        let (function, full_name, link, count, unix, reason) = unsafe {
            (
                row.try_get::<i16>("", "function")
                    .unwrap_unchecked(),
                row.try_get::<String>("", "full_name")
                    .unwrap_unchecked(),
                row.try_get::<String>("", "link")
                    .unwrap_unchecked(),
                row.try_get::<i64>("", "count")
                    .unwrap_unchecked(),
                row.try_get::<i64>("", "removed")
                    .unwrap_unchecked(),
                row.try_get::<String>("", "reason")
                    .unwrap_unchecked(),
            )
        };

        let f = match function {
            1 => "Juzo | Ignore System",
            2 => "Джузо-антиспам",
            _ => unsafe { unreachable() },
        };

        let removed = Local
            .timestamp_opt(unix, 0)
            .unwrap();

        let _ = write!(
            text,
            "\n🗓 Последний вынос из базы <b>«{f}»</b>\n* В сумме <b>{0}</b>.",
            plur_mark(count as u64)
        );

        if !reason.is_empty() {
            let _ = write!(
                text,
                "\n<blockquote expandable><b>Причина: </b>{reason}\n<b>Модератор: </b><a \
                 href='{link}'>{full_name}</a> {0}</blockquote>\n",
                removed.format("в %d.%m.%Y %H:%M")
            );
        } else {
            let _ = write!(
                text,
                "\n<blockquote expandable><b>Модератор: </b><a href='{link}'>{full_name}</a> \
                 {0}</blockquote>\n",
                removed.format("в %d.%m.%Y %H:%M")
            );
        }
    }

    for block in &blocks {
        let (function, full_name, link, added, reason) = unsafe {
            (
                block
                    .try_get::<i16>("", "function")
                    .unwrap_unchecked(),
                block
                    .try_get::<String>("", "full_name")
                    .unwrap_unchecked(),
                block
                    .try_get::<String>("", "link")
                    .unwrap_unchecked(),
                block
                    .try_get::<i64>("", "added")
                    .unwrap_unchecked(),
                block
                    .try_get::<String>("", "reason")
                    .unwrap_unchecked(),
            )
        };

        let added = Local
            .timestamp_opt(added, 0)
            .unwrap();

        let (smail, f) = match function {
            1 => ("🤐", "В режиме игнора команд"),
            2 => ("📛", "Находится в базе «Джузо-антиспам»"),
            _ => unsafe { unreachable() },
        };

        let _ = write!(
            text,
            "\n{smail} <b>{f}</b>\n<blockquote expandable><b>Модератор: </b><a \
             href='{link}'>{full_name}</a> {0}",
            added.format("в %d.%m.%Y %H:%M")
        );

        if !reason.is_empty() {
            let _ = write!(text, "\n<b>Причина: </b>{reason}");
        }

        text.push_str("</blockquote>\n");
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

pub async fn show(
    bot: Bot,
    message: Message,
    db: Extension<DbConn>,
    result: Extension<CommandResult>,
) -> HandlerResult<()> {
    show_core(bot, message, db, result, false).await
}
