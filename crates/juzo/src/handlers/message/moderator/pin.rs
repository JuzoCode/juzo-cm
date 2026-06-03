use juzo_core::common::emojis::smail_tick;
use telers::methods::{PinChatMessage, UnpinChatMessage};

use super::super::*;

pub async fn add(
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
    let args = result.args::<1>(text);

    let message_id: i64 = match args {
        Some([a1]) => {
            let Ok(id) = text[a1].parse::<i64>() else {
                return Ok(());
            };
            id
        }
        None => {
            if let Some(r) = message.reply_to_message() {
                r.message_id()
            } else {
                return Ok(());
            }
        }
    };

    bot.send(
        PinChatMessage::new(message.chat().id(), message_id)
            .business_connection_id_option(message.business_connection_id()),
    )
    .await?;

    Ok(())
}

pub async fn delete(
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
    let args = result.args::<1>(text);

    let message_id: i64 = match args {
        Some([a1]) => {
            let Ok(id) = text[a1].parse::<i64>() else {
                return Ok(());
            };
            id
        }
        None => {
            if let Some(r) = message.reply_to_message() {
                r.message_id()
            } else {
                return Ok(());
            }
        }
    };

    bot.send(
        UnpinChatMessage::new(message.chat().id())
            .message_id(message_id)
            .business_connection_id_option(message.business_connection_id()),
    )
    .await?;

    bot.send(
        JuzoAnswer::message(&message).text(format!("{0} Сообщение открепленно", smail_tick(true))),
    )
    .await?;

    Ok(())
}
