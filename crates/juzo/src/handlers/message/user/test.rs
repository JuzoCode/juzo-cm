use juzo_core::{
    common::emojis::smail_pensil,
    payloads::{
        base::PackedPayload,
        callback::{Callback, CallbackKind},
    },
};
use telers::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use super::super::*;

pub async fn ping(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
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

    bot.send(JuzoAnswer::message(&message).text(format!(
        "<tg-emoji emoji-id='5255971360965930740'>🕓</tg-emoji> Время отправления {label}: \
         <tg-time unix='{time}' format='T'>juzo</tg-time>"
    )))
    .await?;

    Ok(())
}

pub async fn sms_ids(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let id = message
        .reply_to_message()
        .map_or(message.message_id(), |r| r.message_id());

    bot.send(
        JuzoAnswer::message(&message).text(format!("🗓 ID данного сообщения: <code>{id}</code>")),
    )
    .await?;

    Ok(())
}

pub async fn creator_premium_pack(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let Some([a1]) = result.args::<1>(text) else {
        return Ok(());
    };

    let text = match text[a1].parse::<u64>() {
        Ok(num) => format!("🆔 ID создателя пака: <code>@{0}</code>", num >> 32),
        Err(err) => format!("{0} Ошибка при разборе ID: {err}", smail_pensil(true)),
    };

    bot.send(JuzoAnswer::message(&message).text(text))
        .await?;

    Ok(())
}

pub async fn show_thread_link(bot: Bot, message: Message) -> HandlerResult<()> {
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

    let chat_ids_raw = message
        .chat()
        .id()
        .abs()
        .to_string();
    let chat_ids = chat_ids_raw.trim_start_matches("100");

    let url = format!("https://t.me/c/{chat_ids}/{reply_ids}?thread={thread_ids}",);

    let _keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::new("Перейти").url(url)]]);

    //     format!(
    //         "<tg-emoji emoji-id='5229057940543005628'>🧵</tg-emoji> Отдельная ветка <a href='{url_string}'>этого</a> сообщения."
    //     ),

    Ok(())
}
