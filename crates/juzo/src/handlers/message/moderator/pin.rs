use juzo_core::common::emojis::smail_tick;
use telers::methods::{PinChatMessage, UnpinChatMessage};

use super::super::*;

pub async fn add(
    bot: Bot,
    message: Message,
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let module = ModuleChecker::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);

    let message_id: i64 = match args {
        ArgsResult::Some([a1], _) => {
            let Ok(id) = text[a1].parse::<i64>() else {
                return Ok(());
            };
            id
        }
        ArgsResult::None => {
            if let Some(r) = message.reply_to_message() {
                r.message_id()
            } else {
                return Ok(());
            }
        }
        ArgsResult::Unk => return Ok(()),
    };

    let access = module
        .check::<24>(ModuleAccess::M(&message))
        .await;
    if !access {
        return Ok(());
    }

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
    Extension(db): Extension<DbConn>,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    let module = ModuleChecker::new(&bot, &db);

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let args = result.args::<1>(text);

    let message_id: i64 = match args {
        ArgsResult::Some([a1], _) => {
            let Ok(id) = text[a1].parse::<i64>() else {
                return Ok(());
            };
            id
        }
        ArgsResult::None => {
            if let Some(r) = message.reply_to_message() {
                r.message_id()
            } else {
                return Ok(());
            }
        }
        ArgsResult::Unk => return Ok(()),
    };

    let access = module
        .check::<24>(ModuleAccess::M(&message))
        .await;
    if !access {
        return Ok(());
    }

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
