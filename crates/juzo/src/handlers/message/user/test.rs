use juzo_core::{
    common::emojis::smail_pensil,
    domain::ChatIds,
    payloads::{
        base::PackedPayload,
        callback::{Callback, CallbackKind},
    },
};
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

    let module = ModuleChecker::new(&bot, &db);
    let true = module
        .check::<35>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

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

pub async fn chat_ids(
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
        .check::<37>(ModuleAccess::M(&message))
        .await
    else {
        return Ok(());
    };

    bot.send(
        JuzoAnswer::message(&message)
            .text(format!("🗓 ID чата: <code>{0}</code>", message.chat().id())),
    )
    .await?;

    Ok(())
}

pub async fn show_thread_link(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    if let Some(true) = message.chat().is_forum() {
        bot.send(
            JuzoAnswer::message(&message)
                .text(format!("{0} Ветка не сработает с включенными темами.", smail_pensil(true))),
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
                "<tg-emoji emoji-id='5229057940543005628'>🧵</tg-emoji> Отдельная ветка <a \
                 href='{url}'>этого</a> сообщения.<tg-button-row><tg-button type='url' \
                 url='{url}'>Перейти</tg-button></tg-button-row>",
            )))
            .reply_parameters(ReplyParameters::new().message_id(reply_ids)),
    )
    .await?;

    Ok(())
}
