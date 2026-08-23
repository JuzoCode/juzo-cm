use core::fmt::Write;

use chrono::{DateTime, Utc};
use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_pensil,
    db::{
        agent::{BlockFunc, block_system, prelude::BlockSystem},
        chat::block::BlockInfo,
    },
    domain::TimeFormatted,
};
use sea_orm::{EntityTrait, FromQueryResult, QuerySelect, raw_sql};

use super::super::*;

pub async fn info(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);
    let module = ModuleChecker::new(&bot, &db);

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

    let access = module
        .check::<12>(ModuleAccess::M(&message))
        .await;
    if !access {
        return Ok(());
    }

    let chat_ids = message.chat().id();

    let Ok(Some(info)) = BlockInfo::find_by_statement(raw_sql!(
        Postgres,
        r#"
        SELECT
            c8.rank,
            c8.added,
            c8.removed,
            c8.sms_ids,
            c8.reason AS ban_reason,
            c8.moder_ids,
            a3.reason AS spam_reason
        FROM c8
        FULL JOIN a3
            ON a3.user_ids = c8.user_ids
            AND a3.function = 2
        WHERE
            (c8.user_ids = {user.ids}
            AND c8.chat_ids = {chat_ids}
            AND c8.is_ban = true)
            OR
            (a3.user_ids = {user.ids}
            AND a3.function = 2)
            "#
    ))
    .one(&db)
    .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} У <a href='{1}'>{2}</a> не найдено блокировок в Джузо.",
            smail_pensil(true),
            user.link(),
            user.full_name()
        )))
        .await?;

        return Ok(());
    };

    let mut text =
        format!("🗓 <b>Список наказаний <a href='{0}'>{1}</a>.</b>", user.link(), user.full_name());
    if let Some(reason) = info.spam_reason {
        text.push_str("\n* Находится в базе <b>«Джузо-антиспам»</b>");

        if !reason.is_empty() {
            let _ = write!(text, ".<blockquote expandable><b>Причина: </b>{reason}</blockquote>");
        }
    }

    if let Some(reason) = info.ban_reason {
        let removed = unsafe {
            info.removed
                .unwrap_unchecked()
        };
        let added = DateTime::<Utc>::from_timestamp(
            unsafe {
                info.added
                    .unwrap_unchecked()
            },
            0,
        )
        .unwrap_or_default();

        if removed == 0 {
            text.push_str("\n\n<b>❗️ Забанен навсегда");
        } else {
            let _ = write!(text, "\n\n<b>❗️ Забанен на {0}", TimeFormatted::until(removed, added));
        }

        unsafe {
            let _ = write!(
                text,
                " ({0})</b><blockquote expandable><b>Модератор: </b>{1}\n<b>Когда: </b>{2}",
                info.rank
                    .unwrap_unchecked(),
                info.moder_ids
                    .unwrap_unchecked(),
                added.format("%d.%m.%Y")
            );
        }

        if !reason.is_empty() {
            let _ = write!(text, "\n<b>Причина: </b>{reason}");
        }

        text.push_str("</blockquote>")
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

pub async fn scam(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let user_ind = UserIndex::new(&bot, &db);
    let module = ModuleChecker::new(&bot, &db);

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

    let access = module
        .check::<12>(ModuleAccess::M(&message))
        .await;
    if !access {
        return Ok(());
    }

    let Ok(Some((reason, added))) = BlockSystem::find_by_id((user.ids, BlockFunc::Scam))
        .select_only()
        .columns([block_system::Column::Reason, block_system::Column::Added])
        .into_tuple::<(String, i64)>()
        .one(&db)
        .await
    else {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Пользователь не находится в базе скама.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    };

    let mut text = format!(
        "🗓 <a href='{0}'>{1}</a> находится в базе «Juzo | Scam System».\n<blockquote expandable>",
        user.link(),
        user.full_name()
    );

    if !reason.is_empty() {
        let _ = write!(text, "<b>Причина:</b> {reason}\n<b>Добавлен:</b> {added}</blockquote>");
    } else {
        let _ = write!(text, "<b>Добавлен:</b> {added}</blockquote>");
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}
