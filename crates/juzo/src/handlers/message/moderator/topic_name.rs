use juzo_core::common::emojis::smail_pensil;
use telers::methods::{EditForumTopic, EditGeneralForumTopic};

use super::super::*;

pub async fn set(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if result.args.is_empty() {
        return Ok(());
    }

    let Some(true) = message.chat().is_forum() else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Название темы можно устанавливать только в чате с включенными темами.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    // SAFETY: The Command filter will not allow processing of a "None" value.
    let text = unsafe {
        message
            .text()
            .or_else(|| message.caption())
            .unwrap_unchecked()
    };
    let name = &text[result.args];

    if name
        .chars()
        .nth(128)
        .is_some()
    {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Название темы превышает допустимый лимит в 128 символов.",
            smail_pensil(true)
        )))
        .await?;
        return Ok(());
    };

    let is_topic = message
        .is_topic_message()
        .unwrap_or_default();
    let chat_ids = message.chat().id();

    if is_topic {
        unsafe {
            bot.send(
                EditForumTopic::new(
                    chat_ids,
                    message
                        .message_thread_id()
                        .unwrap_unchecked(),
                )
                .name(name),
            )
            .await?
        };
    } else {
        bot.send(EditGeneralForumTopic::new(chat_ids, name))
            .await?;
    }

    Ok(())
}
