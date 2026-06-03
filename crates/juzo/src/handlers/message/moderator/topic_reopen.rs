use juzo_core::common::emojis::smail_pensil;
use telers::methods::{ReopenForumTopic, ReopenGeneralForumTopic};

use super::super::*;

pub async fn set(
    bot: Bot,
    message: Message,
    Extension(result): Extension<CommandResult>,
) -> HandlerResult<()> {
    if !result.args.is_empty() {
        return Ok(());
    }

    let Some(true) = message.chat().is_forum() else {
        bot.send(JuzoAnswer::message(&message).text(format!(
            "{0} Открывать тему можно только в чате с включенными темами.",
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
            bot.send(ReopenForumTopic::new(
                chat_ids,
                message
                    .message_thread_id()
                    .unwrap_unchecked(),
            ))
            .await?
        };
    } else {
        bot.send(ReopenGeneralForumTopic::new(chat_ids))
            .await?;
    }

    Ok(())
}
