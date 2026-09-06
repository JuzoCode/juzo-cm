use core::fmt::Write;

use chrono::{DateTime, Utc};
use juzo_core::{
    application::{UserIndex, UserModel},
    common::emojis::smail_pensil,
    db::{
        agent::{BlockFunc, block_system, prelude::BlockSystem},
        chat::block::BlockInfo,
    },
    domain::{AttachResult, TimeFormatted},
};
use sea_orm::{EntityTrait, FromQueryResult, QuerySelect, raw_sql};
use telers::types::InputRichMessage;

use super::super::*;

pub async fn info(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(arch): Extension<AttachResult>,
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
    let chat_ids = arch.chat_ids;

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

    let true = module
        .check::<12>(ModuleAccess::CustomM(&message, chat_ids.0))
        .await
    else {
        return Ok(());
    };

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
            u.full_name,
            COALESCE(
                'https://t.me/' || u.username,
                'tg://openmessage/user_id=' || c8.moder_ids::text
            ) AS link,
            a3.reason AS spam_reason
        FROM c8
        FULL JOIN a3
            ON a3.user_ids = c8.user_ids
            AND a3.function = 2
        LEFT JOIN u
            ON u.user_ids = c8.moder_ids
        WHERE
            (
                c8.user_ids = {user.ids}
                AND c8.chat_ids = {chat_ids}
                AND c8.is_ban
                AND (
                    c8.removed = 0
                    OR c8.removed > EXTRACT(EPOCH FROM NOW())::bigint
                )
            )
            OR (
                a3.user_ids = {user.ids}
                AND a3.function = 2
            )
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
        text.push_str("<br>* Находится в базе <b>«Джузо-антиспам»</b>");

        if !reason.is_empty() {
            let _ = write!(text, ".<blockquote expandable><b>Причина: </b>{reason}</blockquote>");
        }
    }

    if let Some(reason) = info.ban_reason {
        let removed = unsafe {
            info.removed
                .unwrap_unchecked()
        };
        let sms_ids = unsafe {
            info.sms_ids
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
            let _ = write!(
                text,
                "<br><br><b>❗️ Забанен <tg-button type='url' style='danger' url='https://t.me/c/{0}/{1}'>навсегда",
                chat_ids.some(),
                sms_ids
            );
        } else {
            let _ = write!(
                text,
                "<br><br><b>❗️ Забанен на <tg-button type='url' style='danger' url='https://t.me/c/{0}/{1}'>{2}",
                chat_ids.some(),
                sms_ids,
                TimeFormatted::until(removed, added)
            );
        }

        unsafe {
            let _ = write!(
                text,
                "</tg-button> ({0})</b><blockquote expandable><b>Модератор: </b><a \
                 href='{1}'>{2}</a><br><b>Когда: </b>{3}",
                info.rank
                    .unwrap_unchecked(),
                info.link
                    .unwrap_unchecked(),
                info.full_name
                    .unwrap_unchecked(),
                added.format("%d.%m.%Y")
            );
        }

        if !reason.is_empty() {
            let _ = write!(text, "<br><b>Причина: </b>{reason}");
        }

        text.push_str("</blockquote>")
    }

    bot.send(JuzoAnswer::rich(&message).rich_message(InputRichMessage::new().html(text)))
        .await?;

    Ok(())
}

pub async fn info_mute(
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

    let true = module
        .check::<43>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let chat_ids = message.chat().id();

    let Ok(Some(_info)) = BlockInfo::find_by_statement(raw_sql!(
        Postgres,
        r#"
        SELECT
            rank,
            added,
            removed,
            sms_ids,
            reason AS ban_reason,
            moder_ids
        FROM c8
        WHERE
            (
                c8.user_ids = {user.ids}
                AND c8.chat_ids = {chat_ids}
                AND c8.is_ban = false
                AND (
                    c8.removed = 0
                    OR c8.removed > EXTRACT(EPOCH FROM NOW())::bigint
                )
            )
        "#
    ))
    .one(&db)
    .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} У <a href='{1}'>{2}</a> не заглушён.",
            smail_pensil(true),
            user.link(),
            user.full_name()
        )))
        .await?;
        return Ok(());
    };

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

    if message
        .chat()
        .title()
        .is_some()
    {
        let true = module
            .check::<12>(ModuleAccess::M(&message))
            .await
        else {
            return Ok(());
        };
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

    let added = DateTime::<Utc>::from_timestamp(added, 0).unwrap_or_default();

    if !reason.is_empty() {
        let _ = write!(
            text,
            "<b>Причина:</b> {reason}\n<b>Добавлен:</b> {0}</blockquote>",
            added.format("%d.%m.%Y")
        );
    } else {
        let _ = write!(text, "<b>Добавлен:</b> {0}</blockquote>", added.format("%d.%m.%Y"));
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

pub async fn _warn_list(
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

    let true = module
        .check::<17>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let _chat_ids = message.chat().id();

    let Ok(Some(_info)) = BlockInfo::find_by_statement(raw_sql!(
        Postgres,
        r#"

        "#
    ))
    .one(&db)
    .await
    else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} ❕ <b>Предупреждения <a href='{1}'>{2}</a></b> пока отсутствуют",
            smail_pensil(true),
            user.link(),
            user.full_name()
        )))
        .await?;
        return Ok(());
    };

    Ok(())
}
