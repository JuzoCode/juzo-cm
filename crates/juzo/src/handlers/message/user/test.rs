use core::fmt::Write;

use chrono::{Local, TimeZone};
use juzo_core::{
    common::{emojis::smail_pensil, inflection::plur_mark},
    domain::{ChatIds, UserModel},
    payloads::{
        base::PackedPayload,
        callback::{Callback, CallbackKind},
    },
};
use sea_orm::{ConnectionTrait, raw_sql};
use telers::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, InputRichMessage, ReplyParameters,
};

use super::super::*;

pub async fn ping(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    if message
        .chat()
        .title()
        .is_some()
    {
        let module = ModuleChecker::new(&bot, &db);
        let true = module
            .check::<35>(ModuleAccess::M(&message))
            .await
        else {
            return Ok(());
        };
    }

    // SAFETY: TBA will never return None in message.from().
    let data = unsafe {
        PackedPayload::new(
            message
                .from()
                .unwrap_unchecked()
                .id as u64,
            Callback::new(CallbackKind::Ping),
        )
    };

    let keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::new("Мяу")
        .callback_data(data.encode().as_str())
        .style("primary")]]);

    bot.send(
        JuzoAnswer::message(&message)
            .text("ПОНГ")
            .reply_markup(keyboard),
    )
    .await?;

    Ok(())
}

pub async fn my_spam(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    // SAFETY: TBA will never return None in message.from().
    let iam: UserModel = unsafe {
        message
            .from()
            .unwrap_unchecked()
    }
    .into();

    let antispam = db
        .query_one_raw(raw_sql!(
            Postgres,
            r#"
            SELECT
                a3.reason AS spam_reason,
                a2.removed,
                a2.reason AS take_reason,
                u.full_name,
                COALESCE(
                    'https://t.me/' || u.username,
                    'tg://openmessage/user_id=' || a2.agents_ids::text
                ) AS link,
                a2.count
            FROM (SELECT {iam.ids} AS user_ids) AS base
            LEFT JOIN (
                SELECT
                    removed,
                    reason,
                    agents_ids,
                    COUNT(*) OVER () AS count
                FROM a2
                WHERE user_ids = {iam.ids}
                ORDER BY removed DESC
                LIMIT 1
            ) AS a2
                ON true
            LEFT JOIN a3
                ON a3.user_ids = base.user_ids
                AND a3.function = 2
            LEFT JOIN u
                ON u.user_ids = a2.agents_ids
            "#
        ))
        .await
        .ok()
        .flatten();

    let spam_reason = antispam
        .as_ref()
        .and_then(|row| {
            row.try_get::<String>("", "spam_reason")
                .ok()
        });

    let take_reason = antispam
        .as_ref()
        .and_then(|row| {
            row.try_get::<String>("", "take_reason")
                .ok()
        });

    let mut text = format!("<b>Баны <a href='{0}'>{1}</a>.</b>\n", iam.link(), iam.full_name());
    let mut meow = "\n🗓 Вы абсолютно чисты и <b>не имеете выносов</b> в базе «Джузо-антиспам»";

    if let Some(reason) = spam_reason {
        text.push_str("* В базе <b>«Джузо-антиспам»</b>");
        meow = "\n🗓 Вы <b>не имеете выносов</b> в базе «Джузо-антиспам»";

        if !reason.is_empty() {
            let _ = write!(text, ".<blockquote expandable><b>Причина: </b>{reason}</blockquote>\n");
        } else {
            text.push_str("\n");
        }
    }

    if let Some(reason) = take_reason {
        // SAFETY: take_reason = Some(...) guarantees that antispam = Some(...)
        let row = unsafe { antispam.unwrap_unchecked() };

        let (full_name, link, count, unix) = unsafe {
            (
                row.try_get::<String>("", "full_name")
                    .unwrap_unchecked(),
                row.try_get::<String>("", "link")
                    .unwrap_unchecked(),
                row.try_get::<i64>("", "count")
                    .unwrap_unchecked(),
                row.try_get::<i64>("", "removed")
                    .unwrap_unchecked(),
            )
        };

        let removed = Local
            .timestamp_opt(unix, 0)
            .unwrap();

        let _ = write!(
            text,
            "\n🗓 Последний вынос из базы <b>«Джузо-антиспам»</b>\n* В сумме <b>{0}",
            plur_mark(count as u64)
        );

        if !reason.is_empty() {
            let _ = write!(
                text,
                ".</b>\n<blockquote expandable><b>Причина: </b>{reason}\n<b>Модератор: </b><a \
                 href='{link}'>{full_name}</a> {0}</blockquote>",
                removed.format("в %d.%m.%Y %H:%M")
            );
        } else {
            let _ = write!(
                text,
                ".</b>\n<blockquote expandable><b>Модератор: </b><a href='{link}'>{full_name}</a> \
                 {0}</blockquote>",
                removed.format("в %d.%m.%Y %H:%M")
            );
        }
    } else {
        text.push_str(meow)
    }

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

pub async fn time_sms(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let Some(reply) = message.reply_to_message() else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Для показа времени, ответьте на любое сообщение.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    let (label, time) = reply
        .forward_origin()
        .map_or(("сообщения", reply.date()), |f| ("пересланного сообщения", f.date()));

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!(
                "<tg-emoji emoji-id='5255971360965930740'>🕓</tg-emoji> Время отправления \
                 {label}: <tg-time unix='{time}' format='T'>juzo</tg-time>"
            ))
            .reply_parameters(ReplyParameters::new().message_id(reply.message_id())),
    )
    .await?;

    Ok(())
}

pub async fn sms_ids(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<36>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    let id = message
        .reply_to_message()
        .map_or(message.message_id(), |r| r.message_id());

    bot.send(
        JuzoAnswer::message(&message).text(format!("🗓 ID данного сообщения: <code>{id}</code>")),
    )
    .await?;

    Ok(())
}

pub async fn show_thread_link(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<44>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    if let Some(true) = message.chat().is_forum() {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Ветка не сработает с включёнными темами.", smail_pensil(true))),
        )
        .await?;
        return Ok(());
    };

    let Some(reply) = message.reply_to_message() else {
        bot.send(
            JuzoAnswer::message(&message).text(format!(
                "{0} Вы не сделали ответ ни на какое сообщение.",
                smail_pensil(true)
            )),
        )
        .await?;
        return Ok(());
    };

    let thread_ids = reply
        .message_thread_id()
        .unwrap_or(reply.message_id());
    let reply_ids = reply.message_id();

    let chat_ids = ChatIds(message.chat().id()).some();

    let url = format!("https://t.me/c/{chat_ids}/{reply_ids}?thread={thread_ids}");

    bot.send(
        JuzoAnswer::rich(&message)
            .rich_message(InputRichMessage::new().html(format!(
                "<tg-emoji emoji-id='5229057940543005628'>🧵</tg-emoji> Отдельная ветка \
                 <tg-button type='url' url='{url}'>этого</tg-button> \
                 сообщения.<tg-button-row><tg-button type='url' \
                 url='{url}'>Перейти</tg-button></tg-button-row>",
            )))
            .reply_parameters(ReplyParameters::new().message_id(reply_ids)),
    )
    .await?;

    Ok(())
}
